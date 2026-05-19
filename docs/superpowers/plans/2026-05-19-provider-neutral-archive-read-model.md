# Provider-neutral Archive Read Model Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the first provider-neutral archive/read UI model for source browsing, with source-scoped readiness/versioning, old-path versus new-path parity tests, and gated runtime use that preserves existing behavior until a source has current ready archive rows.

**Architecture:** Keep `items` plus typed provider tables as canonical truth. Add `archive_read_items` as an item-level, provider-neutral, rebuildable read model for source browsing rows and Telegram export-required archive fidelity fields. Add `archive_read_model_state` as the source-scoped gate. Source browsing should use archive rows only when state is `ready` and `model_version` matches the current builder; otherwise it keeps the current provider/archive query path. YouTube transcript segment navigation remains a paired typed reader for this slice, not segment rows in `archive_read_items`.

**Tech Stack:** Rust, SQLx SQLite, Tauri commands, zstd-backed text/media metadata, existing source item query tests, `cargo test`.

---

## File Structure

- Create `src-tauri/migrations/26.sql`: archive read model tables, constraints, and indexes.
- Create `src-tauri/src/archive_read_model.rs`: schema helper, readiness helpers, source rebuild, item upsert/stale helpers, and archive source-browsing query.
- Modify `src-tauri/src/lib.rs`: register the new module.
- Modify `src-tauri/src/migrations.rs`: register migration 26 and include the schema in test migration application.
- Modify `src-tauri/src/sources/mod.rs`: re-export the internal source item row model for the archive reader.
- Modify `src-tauri/src/sources/test_support.rs`: add archive read model fixture helper after the module exists.
- Modify `src-tauri/src/sources/types.rs`: replace raw payload transport in `StoredItemRow` with an explicit `has_raw_data` flag.
- Modify `src-tauri/src/sources/items/query.rs`: preserve current `items` query as the fallback path and add archive-read gating.
- Modify `src-tauri/src/sources/items.rs`: map `has_raw_data`, wire single-write versus bulk-ingest archive maintenance, and keep list DTO behavior unchanged.
- Modify `src-tauri/src/youtube/jobs.rs`: mark affected YouTube sources stale after bulk metadata/transcript/comment refreshes.
- Modify `docs/database-schema.md`, `docs/database-schema-read-model-decision.md`, and `docs/backlog.md`: document the shipped first slice and remaining NotebookLM/export work.

---

### Task 1: Migration 26 And Readiness Schema

**Files:**
- Create: `src-tauri/migrations/26.sql`
- Modify: `src-tauri/src/migrations.rs`
- Test: `src-tauri/src/migrations.rs`

- [x] **Step 1: Add failing migration registration tests**

In `src-tauri/src/migrations.rs`, add:

```rust
#[test]
fn includes_archive_read_model_migration() {
    let migrations = build_migrations();
    let migration = migrations
        .iter()
        .find(|migration| migration.version == 26)
        .expect("version 26 migration is registered");

    assert_eq!(
        migration.description,
        "add provider neutral archive read model"
    );
    for fragment in [
        "CREATE TABLE IF NOT EXISTS archive_read_model_state",
        "CREATE TABLE IF NOT EXISTS archive_read_items",
        "CHECK (status IN ('never_built', 'building', 'ready', 'stale', 'failed'))",
        "idx_archive_read_items_source_published",
        "idx_archive_read_items_source_topic_published",
        "idx_archive_read_items_ref",
    ] {
        assert!(
            migration.sql.contains(fragment),
            "missing migration fragment {fragment}"
        );
    }
}

#[tokio::test]
async fn fresh_schema_includes_archive_read_model_tables_indexes_and_constraints() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");

    apply_all_migrations_for_test_pool(&pool)
        .await
        .expect("apply migrations");

    for table in ["archive_read_model_state", "archive_read_items"] {
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
        "idx_archive_read_items_source_published",
        "idx_archive_read_items_source_topic_published",
        "idx_archive_read_items_ref",
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
}
```

Update the version-list assertion:

```rust
assert_eq!(versions, (1_i64..=26_i64).collect::<Vec<_>>());
```

- [x] **Step 2: Run migration tests and confirm failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::includes_archive_read_model_migration
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::fresh_schema_includes_archive_read_model_tables_indexes_and_constraints
```

Expected: both fail because migration 26 is not registered and the tables do not exist.

- [x] **Step 3: Add migration 26**

Create `src-tauri/migrations/26.sql`:

```sql
CREATE TABLE IF NOT EXISTS archive_read_model_state (
    source_id INTEGER PRIMARY KEY,
    model_version INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'never_built',
    built_at INTEGER,
    item_count INTEGER NOT NULL DEFAULT 0,
    row_count INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    FOREIGN KEY(source_id) REFERENCES sources(id) ON DELETE CASCADE,
    CHECK (status IN ('never_built', 'building', 'ready', 'stale', 'failed')),
    CHECK (item_count >= 0),
    CHECK (row_count >= 0)
);

CREATE TABLE IF NOT EXISTS archive_read_items (
    source_id INTEGER NOT NULL,
    item_id INTEGER NOT NULL,
    ref TEXT NOT NULL,
    external_id TEXT NOT NULL,
    item_kind TEXT NOT NULL,
    author TEXT,
    published_at INTEGER NOT NULL,
    content_kind TEXT NOT NULL,
    has_media INTEGER NOT NULL DEFAULT 0,
    media_kind TEXT,
    content_zstd BLOB,
    media_metadata_zstd BLOB,
    has_raw_data INTEGER NOT NULL DEFAULT 0,
    forum_topic_id INTEGER,
    forum_topic_title TEXT,
    forum_topic_top_message_id INTEGER,
    reply_to_msg_id INTEGER,
    reply_to_peer_kind TEXT,
    reply_to_peer_id TEXT,
    reply_to_top_id INTEGER,
    reaction_count INTEGER,
    model_version INTEGER NOT NULL,
    built_at INTEGER NOT NULL,
    PRIMARY KEY(source_id, item_id),
    FOREIGN KEY(source_id) REFERENCES sources(id) ON DELETE CASCADE,
    FOREIGN KEY(item_id) REFERENCES items(id) ON DELETE CASCADE,
    CHECK (has_media IN (0, 1)),
    CHECK (has_raw_data IN (0, 1))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_archive_read_items_ref
    ON archive_read_items(ref);

CREATE INDEX IF NOT EXISTS idx_archive_read_items_source_published
    ON archive_read_items(source_id, published_at DESC, item_id DESC);

CREATE INDEX IF NOT EXISTS idx_archive_read_items_source_topic_published
    ON archive_read_items(source_id, forum_topic_id, published_at DESC, item_id DESC);
```

This first slice chooses item-level row granularity. It intentionally copies compressed text and compressed media metadata because source browsing renders them, but it does not copy `raw_data_zstd`; raw payload presence is represented by `has_raw_data`.

In `src-tauri/src/migrations.rs`, append:

```rust
Migration {
    version: 26,
    description: "add provider neutral archive read model",
    sql: include_str!("../migrations/26.sql"),
    kind: MigrationKind::Up,
},
```

- [x] **Step 4: Run Task 1 verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::includes_archive_read_model_migration
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::fresh_schema_includes_archive_read_model_tables_indexes_and_constraints
cargo fmt --manifest-path src-tauri/Cargo.toml --check
git diff --check
```

Expected: migration tests pass, formatting passes, and diff check has no whitespace errors except Git line-ending warnings.

- [x] **Step 5: Commit migration schema**

Run:

```powershell
git add src-tauri/migrations/26.sql src-tauri/src/migrations.rs
git commit -m "feat: add archive read model schema"
```

Expected: commit succeeds.

---

### Task 2: Archive Read Model Builder And Readiness Contract

**Files:**
- Create: `src-tauri/src/archive_read_model.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/sources/test_support.rs`
- Test: `src-tauri/src/archive_read_model.rs`
- Test: `src-tauri/src/sources/test_support.rs`

- [x] **Step 1: Register the module and add failing schema tests**

In `src-tauri/src/lib.rs`, add near `mod analysis_documents;`:

```rust
mod archive_read_model;
```

Create `src-tauri/src/archive_read_model.rs` with failing tests first:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::compression::compress_text;
    use crate::sources::test_support::{
        create_analysis_documents_table, create_archive_read_model_tables,
        memory_pool_with_source_items_and_topics,
    };

    #[tokio::test]
    async fn create_schema_adds_state_and_item_tables() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_archive_read_model_tables(&pool).await;

        for table in ["archive_read_model_state", "archive_read_items"] {
            let exists: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?",
            )
            .bind(table)
            .fetch_one(&pool)
            .await
            .expect("check table");
            assert_eq!(exists, 1, "missing table {table}");
        }
    }

    #[tokio::test]
    async fn rebuild_source_materializes_archive_fidelity_fields() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_analysis_documents_table(&pool).await;
        create_archive_read_model_tables(&pool).await;
        seed_archive_source_fixture(&pool).await;

        rebuild_source(&pool, 1).await.expect("rebuild source");

        let row: ArchiveReadItemRow = sqlx::query_as(
            "SELECT * FROM archive_read_items WHERE source_id = 1 AND item_id = 2",
        )
        .fetch_one(&pool)
        .await
        .expect("load archive row");

        assert_eq!(row.ref_, "s1-i2");
        assert_eq!(row.external_id, "701");
        assert_eq!(row.item_kind, "telegram_message");
        assert_eq!(row.forum_topic_id, Some(200));
        assert_eq!(row.forum_topic_title.as_deref(), Some("Roadmap"));
        assert_eq!(row.forum_topic_top_message_id, Some(700));
        assert_eq!(row.reply_to_top_id, Some(200));
        assert_eq!(row.reaction_count, Some(4));
        assert!(row.has_raw_data);
        assert_eq!(row.model_version, ARCHIVE_READ_MODEL_VERSION);

        let state = load_source_state(&pool, 1)
            .await
            .expect("load state")
            .expect("state exists");
        assert_eq!(state.status, STATUS_READY);
        assert_eq!(state.model_version, ARCHIVE_READ_MODEL_VERSION);
        assert_eq!(state.item_count, 2);
        assert_eq!(state.row_count, 2);
        assert!(state.built_at.is_some());
        assert_eq!(state.last_error, None);
    }

    #[tokio::test]
    async fn current_ready_state_rejects_old_model_version() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_archive_read_model_tables(&pool).await;
        sqlx::query(
            "INSERT INTO archive_read_model_state (
                source_id, model_version, status, built_at, item_count, row_count
             ) VALUES (1, ?, 'ready', 100, 1, 1)",
        )
        .bind(ARCHIVE_READ_MODEL_VERSION - 1)
        .execute(&pool)
        .await
        .expect("seed old state");

        assert!(
            !source_archive_model_is_ready(&pool, 1)
                .await
                .expect("check readiness")
        );
    }
}
```

Add a local test helper in the same module:

```rust
async fn seed_archive_source_fixture(pool: &sqlx::SqlitePool) {
    sqlx::query(
        "INSERT OR IGNORE INTO sources (
            id, source_type, source_subtype, external_id, title, is_active, is_member, created_at
         ) VALUES (1, 'telegram', 'supergroup', '12345', 'Forum', 1, 1, 1)",
    )
    .execute(pool)
    .await
    .expect("seed source");

    sqlx::query(
        "INSERT INTO telegram_forum_topics (
            source_id, topic_id, top_message_id, title, last_seen_at, updated_at
         ) VALUES (1, 200, 700, 'Roadmap', 100, 100)",
    )
    .execute(pool)
    .await
    .expect("seed topic");

    sqlx::query(
        "INSERT INTO items (
            id, source_id, external_id, item_kind, author, published_at, ingested_at,
            content_zstd, raw_data_zstd, content_kind, has_media, media_kind,
            media_metadata_zstd, reply_to_top_id, reaction_count
         ) VALUES
           (1, 1, '700', 'telegram_message', 'Ada', 100, 100, ?, ?, 'text_only', 0, NULL, NULL, NULL, NULL),
           (2, 1, '701', 'telegram_message', 'Bob', 101, 101, ?, ?, 'text_only', 0, NULL, NULL, 200, 4)",
    )
    .bind(compress_text("Topic root").expect("compress root"))
    .bind(vec![1_u8])
    .bind(compress_text("Topic reply").expect("compress reply"))
    .bind(vec![2_u8])
    .execute(pool)
    .await
    .expect("seed items");

    for item_id in [1_i64, 2_i64] {
        sqlx::query(
            "INSERT INTO item_topic_memberships (
                item_id, source_id, topic_id, match_kind, resolver_version
             ) VALUES (?, 1, 200, 'fixture', 1)",
        )
        .bind(item_id)
        .execute(pool)
        .await
        .expect("seed membership");
    }
}
```

- [x] **Step 2: Run builder tests and confirm failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml archive_read_model::tests::create_schema_adds_state_and_item_tables
cargo test --manifest-path src-tauri/Cargo.toml archive_read_model::tests::rebuild_source_materializes_archive_fidelity_fields
cargo test --manifest-path src-tauri/Cargo.toml archive_read_model::tests::current_ready_state_rejects_old_model_version
```

Expected: compile failure because the module API does not exist.

- [x] **Step 3: Implement constants, row types, and schema helper**

In `src-tauri/src/archive_read_model.rs`, add:

```rust
use sqlx::{FromRow, SqlitePool};

use crate::error::{AppError, AppResult};
use crate::sources::{ForumTopicFilter, StoredItemRow};

pub(crate) const ARCHIVE_READ_MODEL_VERSION: i64 = 1;
pub(crate) const STATUS_NEVER_BUILT: &str = "never_built";
pub(crate) const STATUS_BUILDING: &str = "building";
pub(crate) const STATUS_READY: &str = "ready";
pub(crate) const STATUS_STALE: &str = "stale";
pub(crate) const STATUS_FAILED: &str = "failed";

pub(crate) const ARCHIVE_READ_MODEL_SCHEMA_SQL: &str = include_str!("../migrations/26.sql");

#[derive(Debug, FromRow)]
pub(crate) struct ArchiveReadModelState {
    pub(crate) source_id: i64,
    pub(crate) model_version: i64,
    pub(crate) status: String,
    pub(crate) built_at: Option<i64>,
    pub(crate) item_count: i64,
    pub(crate) row_count: i64,
    pub(crate) last_error: Option<String>,
    pub(crate) updated_at: i64,
}

#[derive(Debug, FromRow)]
pub(crate) struct ArchiveReadItemRow {
    pub(crate) source_id: i64,
    pub(crate) item_id: i64,
    #[sqlx(rename = "ref")]
    pub(crate) ref_: String,
    pub(crate) external_id: String,
    pub(crate) item_kind: String,
    pub(crate) author: Option<String>,
    pub(crate) published_at: i64,
    pub(crate) content_kind: String,
    pub(crate) has_media: bool,
    pub(crate) media_kind: Option<String>,
    pub(crate) content_zstd: Option<Vec<u8>>,
    pub(crate) media_metadata_zstd: Option<Vec<u8>>,
    pub(crate) has_raw_data: bool,
    pub(crate) forum_topic_id: Option<i64>,
    pub(crate) forum_topic_title: Option<String>,
    pub(crate) forum_topic_top_message_id: Option<i64>,
    pub(crate) reply_to_msg_id: Option<i64>,
    pub(crate) reply_to_peer_kind: Option<String>,
    pub(crate) reply_to_peer_id: Option<String>,
    pub(crate) reply_to_top_id: Option<i64>,
    pub(crate) reaction_count: Option<i64>,
    pub(crate) model_version: i64,
    pub(crate) built_at: i64,
}

pub(crate) async fn create_archive_read_model_schema(pool: &SqlitePool) -> AppResult<()> {
    sqlx::raw_sql(ARCHIVE_READ_MODEL_SCHEMA_SQL)
        .execute(pool)
        .await
        .map_err(AppError::database)?;
    Ok(())
}
```

- [x] **Step 4: Add the test fixture helper**

In `src-tauri/src/sources/test_support.rs`, add:

```rust
pub(crate) async fn create_archive_read_model_tables(pool: &sqlx::SqlitePool) {
    crate::archive_read_model::create_archive_read_model_schema(pool)
        .await
        .expect("create archive read model schema");
}
```

Extend `source_fixture_creates_expected_tables`:

```rust
create_archive_read_model_tables(&pool).await;

"archive_read_model_state",
"archive_read_items",
```

- [x] **Step 5: Implement readiness helpers**

Add:

```rust
pub(crate) async fn load_source_state(
    pool: &SqlitePool,
    source_id: i64,
) -> AppResult<Option<ArchiveReadModelState>> {
    sqlx::query_as(
        "SELECT source_id, model_version, status, built_at, item_count, row_count, last_error, updated_at
         FROM archive_read_model_state
         WHERE source_id = ?",
    )
    .bind(source_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)
}

pub(crate) fn is_current_ready_state(state: Option<&ArchiveReadModelState>) -> bool {
    matches!(
        state,
        Some(state)
            if state.status == STATUS_READY
                && state.model_version == ARCHIVE_READ_MODEL_VERSION
    )
}

pub(crate) async fn source_archive_model_is_ready(
    pool: &SqlitePool,
    source_id: i64,
) -> AppResult<bool> {
    let state = load_source_state(pool, source_id).await?;
    Ok(is_current_ready_state(state.as_ref()))
}
```

- [x] **Step 6: Implement source rebuild**

Implement `rebuild_source(pool, source_id)` with this transaction shape:

```rust
pub(crate) async fn rebuild_source(pool: &SqlitePool, source_id: i64) -> AppResult<()> {
    let started_at = crate::sources::types::now_secs();
    let mut tx = pool.begin().await.map_err(AppError::database)?;
    let result = rebuild_source_in_transaction(&mut tx, source_id, started_at).await;

    match result {
        Ok(()) => {
            tx.commit().await.map_err(AppError::database)?;
            Ok(())
        }
        Err(error) => {
            let _ = tx.rollback().await;
            mark_source_failed(pool, source_id, &error.message).await?;
            Err(error)
        }
    }
}
```

`rebuild_source_in_transaction` should:

1. Upsert `archive_read_model_state` as `building` with current `model_version`.
2. Delete existing `archive_read_items` for `source_id`.
3. Select rows from `items` with the same topic joins as `sources/items/query.rs`.
4. Insert one `archive_read_items` row per `items` row using `ref = format!("s{source_id}-i{item_id}")`.
5. Set state to `ready`, current `model_version`, `built_at`, `item_count`, and `row_count`.

The select should include `items.raw_data_zstd IS NOT NULL AS has_raw_data`; it must not read or copy the raw payload bytes.

- [x] **Step 7: Implement stale/failed helpers**

Add source-scoped helpers for bulk paths and recovery:

```rust
pub(crate) async fn mark_source_stale(pool: &SqlitePool, source_id: i64) -> AppResult<()> {
    let mut conn = pool.acquire().await.map_err(AppError::database)?;
    mark_source_stale_on_connection(&mut conn, source_id).await
}

pub(crate) async fn mark_source_stale_on_connection(
    conn: &mut sqlx::SqliteConnection,
    source_id: i64,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO archive_read_model_state (
            source_id, model_version, status, updated_at
         ) VALUES (?, ?, 'stale', strftime('%s','now'))
         ON CONFLICT(source_id) DO UPDATE SET
            status = CASE
                WHEN archive_read_model_state.status = 'ready' THEN 'stale'
                ELSE archive_read_model_state.status
            END,
            updated_at = strftime('%s','now')",
    )
    .bind(source_id)
    .bind(ARCHIVE_READ_MODEL_VERSION)
    .execute(conn)
    .await
    .map_err(AppError::database)?;
    Ok(())
}
```

`mark_source_failed` is used only after rebuild/backfill failure and must run outside the failed rebuild transaction.

- [x] **Step 8: Run Task 2 verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml archive_read_model::tests::
cargo test --manifest-path src-tauri/Cargo.toml sources::test_support::tests::source_fixture_creates_expected_tables
cargo fmt --manifest-path src-tauri/Cargo.toml --check
git diff --check
```

Expected: archive builder tests pass, source fixture tests pass after helper registration, formatting passes, and diff check has no whitespace errors except Git line-ending warnings.

- [x] **Step 9: Commit builder and readiness contract**

Run:

```powershell
git add src-tauri/src/archive_read_model.rs src-tauri/src/lib.rs src-tauri/src/sources/test_support.rs
git commit -m "feat: build archive read model rows"
```

Expected: commit succeeds.

---

### Task 3: Source Browsing Parity Reader

**Files:**
- Modify: `src-tauri/src/sources/mod.rs`
- Modify: `src-tauri/src/sources/types.rs`
- Modify: `src-tauri/src/sources/items.rs`
- Modify: `src-tauri/src/sources/items/query.rs`
- Modify: `src-tauri/src/archive_read_model.rs`
- Test: `src-tauri/src/sources/items/query.rs`
- Test: `src-tauri/src/archive_read_model.rs`

- [x] **Step 1: Change `StoredItemRow` to carry `has_raw_data`**

In `src-tauri/src/sources/types.rs`, make `StoredItemRow` usable by the crate-local archive reader and comparable in parity tests:

```rust
#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub(crate) struct StoredItemRow {
    pub(crate) id: i64,
    // keep the existing fields crate-visible
}
```

Then replace:

```rust
pub(crate) raw_data_zstd: Option<Vec<u8>>,
```

with:

```rust
pub(crate) has_raw_data: bool,
```

In current source item SQL, replace `items.raw_data_zstd` with:

```sql
items.raw_data_zstd IS NOT NULL AS has_raw_data
```

In `src-tauri/src/sources/items.rs::item_record_from_row`, replace:

```rust
has_raw_data: row.raw_data_zstd.is_some(),
```

with:

```rust
has_raw_data: row.has_raw_data,
```

Update tests that construct `StoredItemRow` query projections to select `items.raw_data_zstd IS NOT NULL AS has_raw_data`.

In `src-tauri/src/sources/mod.rs`, re-export the row type for crate-local read-model code:

```rust
pub(crate) use types::StoredItemRow;
```

- [x] **Step 2: Add old-path versus archive-path parity tests**

In `src-tauri/src/sources/items/query.rs`, add tests that seed the existing source browsing fixture, rebuild archive rows, and compare old/new query results:

```rust
#[tokio::test]
async fn archive_reader_matches_items_path_for_source_browsing_rows() {
    let pool = crate::sources::test_support::memory_pool_with_source_items_and_topics().await;
    crate::sources::test_support::create_archive_read_model_tables(&pool).await;
    seed_browsing_parity_fixture(&pool).await;
    rebuild_source(&pool, 1).await.expect("rebuild archive rows");

    let old_rows = load_item_rows_from_items_path(&pool, 1, 20, None, None, None)
    .await
    .expect("load old path");
    let new_rows = crate::archive_read_model::load_item_rows_from_archive(
        &pool, 1, 20, None, None, None,
    )
    .await
    .expect("load archive path");

    assert_eq!(new_rows, old_rows);
}

#[tokio::test]
async fn archive_reader_matches_topic_filter_and_around_item_semantics() {
    let pool = crate::sources::test_support::memory_pool_with_source_items_and_topics().await;
    crate::sources::test_support::create_archive_read_model_tables(&pool).await;
    seed_browsing_parity_fixture(&pool).await;
    rebuild_source(&pool, 1).await.expect("rebuild archive rows");

    for filter in [
        Some(ForumTopicFilter::Topic { topic_id: 200 }),
        Some(ForumTopicFilter::Uncategorized),
        None,
    ] {
        let old_rows = load_item_rows_from_items_path(&pool, 1, 2, None, filter.clone(), Some(11))
        .await
        .expect("load old path");
        let new_rows = crate::archive_read_model::load_item_rows_from_archive(
            &pool, 1, 2, None, filter, Some(11),
        )
        .await
        .expect("load archive path");

        assert_eq!(new_rows, old_rows);
    }
}
```

Keep comparison at the row model level first; `ItemRecord` mapping is covered by existing command tests.

- [x] **Step 3: Run parity tests and confirm failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::items::query::tests::archive_reader_matches_items_path_for_source_browsing_rows
cargo test --manifest-path src-tauri/Cargo.toml sources::items::query::tests::archive_reader_matches_topic_filter_and_around_item_semantics
```

Expected: fails because the archive reader does not exist.

- [x] **Step 4: Preserve current query as explicit old path**

In `src-tauri/src/sources/items/query.rs`, rename the current function to:

```rust
pub(crate) async fn load_item_rows_from_items_path(
    pool: &sqlx::SqlitePool,
    source_id: i64,
    limit: i64,
    before_published_at: Option<i64>,
    topic_filter: Option<ForumTopicFilter>,
    around_item_id: Option<i64>,
) -> AppResult<Vec<StoredItemRow>> {
    // existing implementation
}
```

Keep test names and assertions, but call `load_item_rows_from_items_path` until the gated wrapper is introduced in Task 5.

- [x] **Step 5: Implement archive reader with identical paging/filter semantics**

In `src-tauri/src/archive_read_model.rs`, add:

```rust
pub(crate) async fn load_item_rows_from_archive(
    pool: &SqlitePool,
    source_id: i64,
    limit: i64,
    before_published_at: Option<i64>,
    topic_filter: Option<ForumTopicFilter>,
    around_item_id: Option<i64>,
) -> AppResult<Vec<StoredItemRow>> {
    let around_published_at = if let Some(item_id) = around_item_id {
        sqlx::query_scalar::<_, i64>(
            "SELECT published_at
             FROM archive_read_items
             WHERE source_id = ? AND item_id = ? AND model_version = ?
             LIMIT 1",
        )
        .bind(source_id)
        .bind(item_id)
        .bind(ARCHIVE_READ_MODEL_VERSION)
        .fetch_optional(pool)
        .await
        .map_err(AppError::database)?
    } else {
        None
    };

    let state = crate::topic_memberships::load_topic_resolution_state(pool, source_id).await?;
    let is_ready = crate::topic_memberships::is_ready_current_state(state.as_ref());
    if matches!(
        topic_filter.as_ref(),
        Some(ForumTopicFilter::Uncategorized)
    ) && !is_ready {
        return Ok(Vec::new());
    }

    let mut sql = String::from(
        r#"
        SELECT
            item_id AS id,
            source_id,
            external_id,
            item_kind,
            author,
            published_at,
            content_kind,
            has_media,
            media_kind,
            content_zstd,
            media_metadata_zstd,
            has_raw_data,
            reply_to_msg_id,
            reply_to_peer_kind,
            reply_to_peer_id,
            reply_to_top_id,
            reaction_count,
            forum_topic_id,
            forum_topic_title,
            forum_topic_top_message_id
        FROM archive_read_items
        WHERE source_id = ?
          AND model_version = ?
        "#,
    );

    if before_published_at.is_some() {
        sql.push_str(" AND published_at < ?");
    } else if around_published_at.is_some() {
        sql.push_str(" AND published_at <= ?");
    }

    match topic_filter.as_ref() {
        Some(ForumTopicFilter::Topic { .. }) => {
            sql.push_str(" AND forum_topic_id = ?");
        }
        Some(ForumTopicFilter::Uncategorized) => {
            sql.push_str(" AND forum_topic_id IS NULL");
        }
        None => {}
    }

    sql.push_str(" ORDER BY published_at DESC LIMIT ?");

    let mut query = sqlx::query_as::<_, StoredItemRow>(&sql)
        .bind(source_id)
        .bind(ARCHIVE_READ_MODEL_VERSION);
    if let Some(before) = before_published_at {
        query = query.bind(before);
    } else if let Some(around) = around_published_at {
        query = query.bind(around);
    }
    if let Some(ForumTopicFilter::Topic { topic_id }) = topic_filter.as_ref() {
        query = query.bind(*topic_id);
    }

    query
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(AppError::database)
}
```

- [x] **Step 6: Run Task 3 verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::items::query::tests::
cargo test --manifest-path src-tauri/Cargo.toml sources::items::query::tests::archive_reader_matches_
cargo fmt --manifest-path src-tauri/Cargo.toml --check
git diff --check
```

Expected: old query tests pass, archive parity tests pass, formatting passes, and diff check has no whitespace errors except Git line-ending warnings.

- [x] **Step 7: Commit parity reader**

Run:

```powershell
git add src-tauri/src/sources/mod.rs src-tauri/src/sources/types.rs src-tauri/src/sources/items.rs src-tauri/src/sources/items/query.rs src-tauri/src/archive_read_model.rs
git commit -m "feat: add archive source browsing reader"
```

Expected: commit succeeds.

---

### Task 4: Write Maintenance And Bulk-Ingest Staleness

**Files:**
- Modify: `src-tauri/src/archive_read_model.rs`
- Modify: `src-tauri/src/sources/items.rs`
- Modify: `src-tauri/src/youtube/jobs.rs`
- Test: `src-tauri/src/archive_read_model.rs`
- Test: `src-tauri/src/sources/items.rs`

- [x] **Step 1: Add failing single-write maintenance tests**

In `src-tauri/src/sources/items.rs`, add:

```rust
#[tokio::test]
async fn single_telegram_insert_maintains_ready_archive_model() {
    let pool = memory_pool_with_source_items_and_topics().await;
    create_analysis_documents_table(&pool).await;
    crate::sources::test_support::create_archive_read_model_tables(&pool).await;
    seed_item_source(&pool, 1).await;
    crate::archive_read_model::rebuild_source(&pool, 1)
        .await
        .expect("initial rebuild");

    assert!(
        insert_telegram_source_item(
            &pool,
            1,
            telegram_identity(42),
            telegram_insert("42", "new ready archive row"),
        )
        .await
        .expect("insert item")
    );

    let exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM archive_read_items WHERE source_id = 1 AND ref = 's1-i1'",
    )
    .fetch_one(&pool)
    .await
    .expect("count archive row");
    assert_eq!(exists, 1);

    assert!(
        crate::archive_read_model::source_archive_model_is_ready(&pool, 1)
            .await
            .expect("ready check")
    );
}
```

If item ids are not deterministic in the fixture, load the inserted `item_id` from the insert outcome and assert `ref = format!("s1-i{item_id}")`.

- [x] **Step 2: Add failing bulk-ingest stale test**

In `src-tauri/src/sources/items.rs`, add:

```rust
#[tokio::test]
async fn takeout_observation_insert_marks_ready_archive_model_stale_without_per_item_build() {
    let pool = memory_pool_with_source_items_and_topics().await;
    create_analysis_documents_table(&pool).await;
    crate::sources::test_support::create_ingest_provenance_tables(&pool).await;
    crate::sources::test_support::create_archive_read_model_tables(&pool).await;
    seed_item_source(&pool, 1).await;
    crate::archive_read_model::rebuild_source(&pool, 1)
        .await
        .expect("initial rebuild");
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

    let outcome = insert_telegram_source_item_with_observation(
        &pool,
        batch_id,
        1,
        telegram_identity(77),
        telegram_insert("77", "bulk row"),
    )
    .await
    .expect("bulk insert");

    assert!(outcome.is_inserted());
    let state = crate::archive_read_model::load_source_state(&pool, 1)
        .await
        .expect("load state")
        .expect("state exists");
    assert_eq!(state.status, crate::archive_read_model::STATUS_STALE);
}
```

This locks the important decision: Takeout/bulk ingest does not let archive builder defects roll back large canonical import batches.

- [x] **Step 3: Run maintenance tests and confirm failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::items::tests::single_telegram_insert_maintains_ready_archive_model
cargo test --manifest-path src-tauri/Cargo.toml sources::items::tests::takeout_observation_insert_marks_ready_archive_model_stale_without_per_item_build
```

Expected: fail because write maintenance is not wired.

- [x] **Step 4: Implement item-level archive upsert**

In `src-tauri/src/archive_read_model.rs`, add:

```rust
pub(crate) async fn upsert_item_on_connection(
    conn: &mut sqlx::SqliteConnection,
    item_id: i64,
) -> AppResult<()> {
    let source_id: i64 = sqlx::query_scalar("SELECT source_id FROM items WHERE id = ?")
        .bind(item_id)
        .fetch_one(&mut *conn)
        .await
        .map_err(AppError::database)?;

    let state: Option<ArchiveReadModelState> = sqlx::query_as(
        "SELECT source_id, model_version, status, built_at, item_count, row_count, last_error, updated_at
         FROM archive_read_model_state
         WHERE source_id = ?",
    )
    .bind(source_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(AppError::database)?;

    if !is_current_ready_state(state.as_ref()) {
        return mark_source_stale_on_connection(conn, source_id).await;
    }

    let built_at = crate::sources::types::now_secs();
    let row = load_builder_row_for_item(conn, item_id).await?;
    upsert_archive_row_on_connection(conn, row, built_at).await?;
    refresh_ready_counts_on_connection(conn, source_id, built_at).await
}
```

The helper must run inside the caller transaction. For a single write, failure rolls back the whole canonical item insert. For non-ready sources, it only marks stale/no-ready state and does not force a rebuild.

- [x] **Step 5: Distinguish single write from bulk observation insert**

In `src-tauri/src/sources/items.rs`, add:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ArchiveReadMaintenanceMode {
    MaintainSingleWrite,
    MarkSourceStaleOnly,
}
```

Change `insert_telegram_source_item_on_connection` signature to accept the mode:

```rust
async fn insert_telegram_source_item_on_connection(
    conn: &mut sqlx::SqliteConnection,
    source_id: i64,
    identity: TelegramMessageIdentity,
    item: SourceItemInsert,
    archive_maintenance: ArchiveReadMaintenanceMode,
) -> AppResult<TelegramItemInsertOutcome>
```

After `analysis_documents::upsert_item_backed_document_on_connection`, add:

```rust
match archive_maintenance {
    ArchiveReadMaintenanceMode::MaintainSingleWrite => {
        crate::archive_read_model::upsert_item_on_connection(conn, item_id).await?;
    }
    ArchiveReadMaintenanceMode::MarkSourceStaleOnly => {
        crate::archive_read_model::mark_source_stale_on_connection(conn, source_id).await?;
    }
}
```

Call with:

```rust
ArchiveReadMaintenanceMode::MaintainSingleWrite
```

from `insert_telegram_source_item_outcome`, and:

```rust
ArchiveReadMaintenanceMode::MarkSourceStaleOnly
```

from `insert_telegram_source_item_with_observation`.

- [x] **Step 6: Mark YouTube bulk refreshes stale**

In `src-tauri/src/youtube/jobs.rs`, mark affected source scopes stale after successful transcript/comment/metadata refreshes that can alter item or display rows:

```rust
crate::archive_read_model::mark_source_stale(&pool, sync_source_id).await?;
```

Do this at job/scope boundaries, not inside every transcript/comment row upsert. Keep canonical YouTube item writes governed by their existing transaction policy.

- [x] **Step 7: Run Task 4 verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::items::tests::single_telegram_insert_maintains_ready_archive_model
cargo test --manifest-path src-tauri/Cargo.toml sources::items::tests::takeout_observation_insert_marks_ready_archive_model_stale_without_per_item_build
cargo test --manifest-path src-tauri/Cargo.toml sources::items::tests::
cargo test --manifest-path src-tauri/Cargo.toml archive_read_model::tests::
cargo fmt --manifest-path src-tauri/Cargo.toml --check
git diff --check
```

Expected: maintenance tests pass, existing item tests pass, archive tests pass, formatting passes, and diff check has no whitespace errors except Git line-ending warnings.

- [x] **Step 8: Commit maintenance semantics**

Run:

```powershell
git add src-tauri/src/archive_read_model.rs src-tauri/src/sources/items.rs src-tauri/src/youtube/jobs.rs
git commit -m "feat: maintain archive read model readiness"
```

Expected: commit succeeds.

---

### Task 5: Gated Source Browsing Switch

**Files:**
- Modify: `src-tauri/src/sources/items/query.rs`
- Modify: `src-tauri/src/sources/items.rs`
- Modify: `src-tauri/src/archive_read_model.rs`
- Test: `src-tauri/src/sources/items/query.rs`

- [x] **Step 1: Add failing gated read-path tests**

In `src-tauri/src/sources/items/query.rs`, add:

```rust
#[tokio::test]
async fn load_item_rows_uses_items_path_when_archive_model_is_not_ready() {
    let pool = memory_pool_with_source_items_and_topics().await;
    seed_source_browsing_fixture(&pool).await;

    let rows = load_item_rows_from_pool(&pool, 1, 20, None, None, None)
        .await
        .expect("load fallback rows");

    assert_eq!(rows.len(), 5);
    assert_eq!(rows[0].external_id, "not-numeric-root");
}

#[tokio::test]
async fn load_item_rows_uses_archive_path_when_ready_and_current() {
    let pool = memory_pool_with_source_items_and_topics().await;
    crate::sources::test_support::create_archive_read_model_tables(&pool).await;
    seed_source_browsing_fixture(&pool).await;
    crate::archive_read_model::rebuild_source(&pool, 1)
        .await
        .expect("rebuild archive rows");

    sqlx::query("DELETE FROM items WHERE source_id = 1 AND external_id = 'not-numeric-root'")
        .execute(&pool)
        .await
        .expect("delete canonical row after archive build");

    let rows = load_item_rows_from_pool(&pool, 1, 20, None, None, None)
        .await
        .expect("load archive rows");

    assert_eq!(rows.len(), 5);
    assert_eq!(rows[0].external_id, "not-numeric-root");
}

#[tokio::test]
async fn load_item_rows_uses_items_path_when_archive_model_is_stale() {
    let pool = memory_pool_with_source_items_and_topics().await;
    crate::sources::test_support::create_archive_read_model_tables(&pool).await;
    seed_source_browsing_fixture(&pool).await;
    crate::archive_read_model::rebuild_source(&pool, 1)
        .await
        .expect("rebuild archive rows");
    crate::archive_read_model::mark_source_stale(&pool, 1)
        .await
        .expect("mark stale");

    sqlx::query("DELETE FROM archive_read_items WHERE source_id = 1")
        .execute(&pool)
        .await
        .expect("delete archive rows");

    let rows = load_item_rows_from_pool(&pool, 1, 20, None, None, None)
        .await
        .expect("load fallback rows");

    assert_eq!(rows.len(), 5);
}
```

Extract the fixture body from `load_item_rows_attaches_topic_metadata_and_root_matches` into `seed_source_browsing_fixture(&pool)` so old query tests and gated tests share exactly the same data.

- [x] **Step 2: Run gated tests and confirm failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::items::query::tests::load_item_rows_uses_items_path_when_archive_model_is_not_ready
cargo test --manifest-path src-tauri/Cargo.toml sources::items::query::tests::load_item_rows_uses_archive_path_when_ready_and_current
cargo test --manifest-path src-tauri/Cargo.toml sources::items::query::tests::load_item_rows_uses_items_path_when_archive_model_is_stale
```

Expected: compile or behavior failure because the wrapper does not exist yet.

- [x] **Step 3: Implement the readiness gate**

In `src-tauri/src/sources/items/query.rs`, reintroduce `load_item_rows_from_pool` as a wrapper:

```rust
pub(super) async fn load_item_rows_from_pool(
    pool: &sqlx::SqlitePool,
    source_id: i64,
    limit: i64,
    before_published_at: Option<i64>,
    topic_filter: Option<ForumTopicFilter>,
    around_item_id: Option<i64>,
) -> AppResult<Vec<StoredItemRow>> {
    if crate::archive_read_model::source_archive_model_is_ready(pool, source_id).await? {
        return crate::archive_read_model::load_item_rows_from_archive(
            pool,
            source_id,
            limit,
            before_published_at,
            topic_filter,
            around_item_id,
        )
        .await;
    }

    load_item_rows_from_items_path(
        pool,
        source_id,
        limit,
        before_published_at,
        topic_filter,
        around_item_id,
    )
    .await
}
```

This is the runtime switch for the first consumer, but it is gated. Missing, stale, failed, building, or old-version source states keep the old provider/archive path.

- [x] **Step 4: Run Task 5 verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::items::query::tests::
cargo test --manifest-path src-tauri/Cargo.toml sources::items::tests::
cargo test --manifest-path src-tauri/Cargo.toml archive_read_model::tests::
cargo fmt --manifest-path src-tauri/Cargo.toml --check
git diff --check
```

Expected: source browsing query tests, item tests, and archive tests pass; formatting and diff check pass.

- [x] **Step 5: Commit gated source browsing switch**

Run:

```powershell
git add src-tauri/src/sources/items/query.rs src-tauri/src/sources/items.rs src-tauri/src/archive_read_model.rs
git commit -m "feat: gate source browsing on archive read model"
```

Expected: commit succeeds.

---

### Task 6: Documentation And Full Verification

**Files:**
- Modify: `docs/database-schema.md`
- Modify: `docs/database-schema-read-model-decision.md`
- Modify: `docs/backlog.md`
- Modify: `docs/superpowers/plans/2026-05-19-provider-neutral-archive-read-model.md`
- Test: full Rust suite

- [x] **Step 1: Update schema docs**

In `docs/database-schema.md`, add sections for:

```markdown
### `archive_read_model_state`

Source-scoped readiness gate for the provider-neutral archive/read UI model.
`items` and typed provider tables remain canonical; this table only decides
whether consumers may use derived archive rows for a source.

- `source_id`: source scope and primary key.
- `model_version`: current archive builder contract version used for this
  source.
- `status`: `never_built`, `building`, `ready`, `stale`, or `failed`.
- `built_at`: timestamp for the successful ready build.
- `item_count` / `row_count`: rebuild accounting for diagnostics.
- `last_error`: bounded rebuild/backfill error, not canonical provider data.

### `archive_read_items`

Provider-neutral item-level archive rows for source browsing. The table
duplicates compressed text and compressed media metadata needed for display,
but does not duplicate `items.raw_data_zstd`; raw payload availability is
represented by `has_raw_data`.
```

Update migration history:

```markdown
| 26 | `26.sql` | Add provider-neutral archive read model tables |
```

- [x] **Step 2: Update the decision note and backlog**

In `docs/database-schema-read-model-decision.md`, update follow-up status to show that the first implementation slice chose:

- `archive_read_items` item-level rows;
- source-scoped `archive_read_model_state`;
- current builder `model_version = 1`;
- gated source browsing as the first consumer;
- YouTube transcript segments remaining on a paired typed segment reader;
- NotebookLM export migration still pending.

In `docs/backlog.md`, move the next Database schema simplification work to:

```markdown
- [ ] migrate NotebookLM export to archive read model after export parity tests
- [ ] decide whether future YouTube playlist-entry browsing needs archive rows or typed detail only
- [ ] consider current-schema baseline after archive read model boundary stabilizes
```

- [x] **Step 3: Mark plan checkboxes complete as work lands**

Before the docs commit, update this plan so completed steps use `[x]`. Do not mark a step complete before its verification command has passed.

- [x] **Step 4: Run containment scans**

Run:

```powershell
rg -n "archive_read_model|archive_read_items|archive_read_model_state" src-tauri/src docs
rg -n "raw_data_zstd" src-tauri/src/sources src-tauri/src/archive_read_model.rs
rg -n "NotebookLM|notebooklm|export_source_to_notebooklm" src-tauri/src docs/database-schema-read-model-decision.md
rg -n "T[O]DO|T[B]D|FIX[M]E|todo!\\(" src-tauri/src docs
```

Expected:

- archive model references are limited to the new module, migration registration, source browsing gate, write maintenance, and docs;
- source browsing no longer transports raw payload blobs just to compute `has_raw_data`;
- NotebookLM export code remains on the old path in this slice;
- no unfinished-work markers or Rust incomplete macro calls are introduced.

- [x] **Step 5: Run targeted Rust tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::includes_archive_read_model_migration
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::fresh_schema_includes_archive_read_model_tables_indexes_and_constraints
cargo test --manifest-path src-tauri/Cargo.toml archive_read_model::tests::
cargo test --manifest-path src-tauri/Cargo.toml sources::items::query::tests::
cargo test --manifest-path src-tauri/Cargo.toml sources::items::tests::
cargo test --manifest-path src-tauri/Cargo.toml youtube::jobs::tests::
```

Expected: all targeted tests pass.

- [x] **Step 6: Run full verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml --check
git diff --check
git status --short
```

Expected: Rust suite and formatting pass. `git diff --check` has no whitespace errors except Git line-ending warnings. `git status --short` shows only intended docs/plan changes before the docs commit.

- [x] **Step 7: Commit docs and final plan state**

Run:

```powershell
git add docs/database-schema.md docs/database-schema-read-model-decision.md docs/backlog.md docs/superpowers/plans/2026-05-19-provider-neutral-archive-read-model.md
git commit -m "docs: document archive read model implementation"
```

Expected: commit succeeds.

---

## Self-Review Checklist

- Decision coverage: the plan implements the selected separate archive/read UI model and keeps `analysis_documents` untouched.
- Scope control: source browsing is the first consumer; NotebookLM export remains pending and receives no runtime migration in this slice.
- Row granularity: `archive_read_items` is item-level. YouTube transcript segment navigation remains the existing paired typed reader for this slice.
- Readiness contract: consumers use archive rows only for `ready` sources whose `model_version` matches `ARCHIVE_READ_MODEL_VERSION`.
- Version invalidation: any future row-shape, derived-field, filtering, or backfill-correctness change must bump `ARCHIVE_READ_MODEL_VERSION`.
- Bulk ingest safety: single writes maintain archive rows synchronously and roll back on builder failure; Takeout/sync-style bulk paths mark source readiness stale instead of per-item archive rollback.
- Raw payload handling: archive rows store `has_raw_data`, not `raw_data_zstd`.
- Parity gate: source browsing old-path versus archive-path output comparison is required before the gated switch.
- Runtime behavior: missing/stale/failed archive readiness falls back to the existing provider/archive path, so existing users do not depend on a first-run full backfill.
- Documentation: schema docs, decision note, and backlog must identify NotebookLM export migration as the next separate slice.
